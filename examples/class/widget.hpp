#pragma once
// C++ class header for the cpp2rust-demo class example.

class Widget {
public:
    /// Construct a widget with a given ID.
    Widget(int id);
    ~Widget();

    /// Update the widget's position.
    void update(double x, double y);

    /// Get the widget's ID (const method).
    int getId() const;

    /// Check whether the widget is visible (const method).
    bool isVisible() const;

    /// Global count of Widget instances (static method).
    static int instanceCount();

private:
    int id_;
    double x_ = 0.0;
    double y_ = 0.0;
    bool visible_ = true;
};
