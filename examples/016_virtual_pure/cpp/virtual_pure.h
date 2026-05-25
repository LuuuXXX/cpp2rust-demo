#pragma once

#ifdef __cplusplus
extern "C" {
#endif

struct AbstractShape;

struct AbstractShape* abstract_shape_create_circle(double radius);
struct AbstractShape* abstract_shape_create_rectangle(double width, double height);
void abstract_shape_delete(struct AbstractShape* self);

double abstract_shape_area(struct AbstractShape* self);
const char* abstract_shape_getName(struct AbstractShape* self);

#ifdef __cplusplus
}

// Full class definition - for hicc code generation
class AbstractShape {
public:
    virtual ~AbstractShape() = default;
    virtual double area() const = 0;
    virtual const char* getName() const = 0;
};

class Circle : public AbstractShape {
    double radius;
public:
    Circle(double r);
    ~Circle() override;
    double area() const override;
    const char* getName() const override;
};

class Rectangle : public AbstractShape {
    double width;
    double height;
public:
    Rectangle(double w, double h);
    ~Rectangle() override;
    double area() const override;
    const char* getName() const override;
};

#endif
